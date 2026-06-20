<details>
<summary>XSD contract: <code>DataKeyValueType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="DataKeyValueType">
		<xs:annotation>
			<xs:documentation>DataKeyValueType is a type for providing one or more dimension values for the purpose of defining a distinct data key.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="MemberSelectionType">
				<xs:sequence>
					<xs:annotation>
						<xs:documentation>v3.1  allows for unbounded Values to support use case with keys that define multiple values for a Dimension i.e. FREQ=A or M or Q  and REF_AREA=DE and CURR=EUR</xs:documentation>
					</xs:annotation>
					<xs:element name="Value" type="SimpleKeyValueType" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="id" type="common:SingleNCNameIDType" use="required"/>
				<xs:attribute name="include" type="xs:boolean" use="optional" fixed="true"/>
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="prohibited"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="prohibited"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
