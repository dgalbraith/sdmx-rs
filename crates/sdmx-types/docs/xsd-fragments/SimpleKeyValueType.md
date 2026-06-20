<details>
<summary>XSD contract: <code>SimpleKeyValueType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="SimpleKeyValueType">
		<xs:annotation>
			<xs:documentation>SimpleKeyValueType derives from the SimpleValueType, but does not allow for the cascading of value in the hierarchy, as keys are meant to describe a distinct full or partial key.</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:restriction base="SimpleComponentValueType">
				<xs:attribute name="cascadeValues" type="common:CascadeSelectionType" use="prohibited"/>
				<xs:attribute ref="xml:lang" use="prohibited"/>
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="prohibited"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="prohibited"/>
			</xs:restriction>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
