<details>
<summary>XSD contract: <code>DataComponentValueType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="DataComponentValueType">
		<xs:annotation>
			<xs:documentation>DataComponentValueType derives from the SimpleValueType, but does not allow for validity dates.</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:restriction base="SimpleComponentValueType">
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="prohibited"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="prohibited"/>
			</xs:restriction>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
